<details>
<summary>XSD contract: <code>MeasureListType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="MeasureListType">
		<xs:annotation>
			<xs:documentation>MeasureListType describes the structure of the measure descriptor for a data structure definition.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="ComponentListType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element ref="common:Link" minOccurs="0" maxOccurs="unbounded"/>
					<xs:sequence maxOccurs="unbounded">
						<xs:element ref="Measure"/>
					</xs:sequence>
				</xs:sequence>
				<xs:attribute name="urn" type="common:MeasureDescriptorUrnType" use="optional"/>
				<xs:attribute name="id" type="common:IDType" use="optional" fixed="MeasureDescriptor"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
