<details>
<summary>XSD contract: <code>DataConstraintAttachmentType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="DataConstraintAttachmentType">
		<xs:annotation>
			<xs:documentation>DataConstraintAttachmentType restricts the base ConstraintAttachmentType to only allow artefacts related to data.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="ConstraintAttachmentType">
				<xs:choice>
					<xs:element name="DataProvider" type="common:DataProviderReferenceType"/>
					<xs:element name="DataStructure" type="common:DataStructureReferenceType" maxOccurs="unbounded"/>
					<xs:element name="Dataflow" type="common:DataflowReferenceType" maxOccurs="unbounded"/>
					<xs:element name="ProvisionAgreement" type="common:ProvisionAgreementReferenceType" maxOccurs="unbounded"/>				</xs:choice>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
