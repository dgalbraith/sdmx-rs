<details>
<summary>XSD contract: <code>DataConstraintAttachmentType</code> (SDMX 3.0)</summary>

```xml
	<xs:complexType name="DataConstraintAttachmentType">
		<xs:annotation>
			<xs:documentation>DataConstraintAttachmentType restricts the base ConstraintAttachmentType to only allow artefacts related to data.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="ConstraintAttachmentType">
				<xs:choice>
					<xs:element name="DataProvider" type="common:DataProviderReferenceType"/>
					<xs:element name="SimpleDataSource" type="xs:anyURI" maxOccurs="unbounded"/>
					<xs:choice>
						<xs:sequence>
							<xs:element name="DataStructure" type="common:DataStructureReferenceType" maxOccurs="unbounded"/>
							<xs:element name="QueryableDataSource" type="common:QueryableDataSourceType" minOccurs="0" maxOccurs="unbounded"/>
						</xs:sequence>
						<xs:sequence>
							<xs:element name="Dataflow" type="common:DataflowReferenceType" maxOccurs="unbounded"/>
							<xs:element name="QueryableDataSource" type="common:QueryableDataSourceType" minOccurs="0" maxOccurs="unbounded"/>
						</xs:sequence>
						<xs:sequence>
							<xs:element name="ProvisionAgreement" type="common:ProvisionAgreementReferenceType" maxOccurs="unbounded"/>
							<xs:element name="QueryableDataSource" type="common:QueryableDataSourceType" minOccurs="0" maxOccurs="unbounded"/>
						</xs:sequence>
					</xs:choice>
				</xs:choice>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
